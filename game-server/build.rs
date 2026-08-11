use std::{env, error::Error, fmt, fmt::Write, fs, path::PathBuf, thread};

const MAGIC_ROOT_SEED: u64 = 0x90fafe6a82f2632f;

#[derive(Clone, Copy)]
enum Slider {
    Rook,
    Bishop,
}

#[derive(Clone, Copy, Default)]
struct MagicEntry {
    mask: u64,
    magic: u64,
    shift: u32,
    offset: usize,
}

impl Slider {
    fn directions(self) -> [(i32, i32); 4] {
        match self {
            Slider::Rook => [(1, 0), (-1, 0), (0, 1), (0, -1)],
            Slider::Bishop => [(1, 1), (1, -1), (-1, 1), (-1, -1)],
        }
    }
}

impl fmt::Display for MagicEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MagicEntry {{ mask: {:#018x}, magic: {:#018x}, shift: {}, offset: {} }}",
            self.mask, self.magic, self.shift, self.offset
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let rook_handle = thread::spawn(generate_rook_output);
    let bishop_handle = thread::spawn(generate_bishop_output);

    let rook_output = rook_handle.join().unwrap()?;
    let bishop_output = bishop_handle.join().unwrap()?;

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let rook_path = out_dir.join("rook_magics.rs");
    let bishop_path = out_dir.join("bishop_magics.rs");

    let rook_handle = thread::spawn(move || fs::write(rook_path, rook_output));
    let bishop_handle = thread::spawn(move || fs::write(bishop_path, bishop_output));

    rook_handle.join().unwrap()?;
    bishop_handle.join().unwrap()?;

    Ok(())
}

fn generate_rook_output() -> Result<String, fmt::Error> {
    let (magics, attacks) = generate_slider_tables(Slider::Rook);
    let mut output = String::new();

    write_magics(&mut output, "ROOK_MAGICS", &magics)?;
    write_attacks(&mut output, "ROOK_ATTACKS", &attacks)?;

    Ok(output)
}

fn generate_bishop_output() -> Result<String, fmt::Error> {
    let (magics, attacks) = generate_slider_tables(Slider::Bishop);
    let mut output = String::new();

    write_magics(&mut output, "BISHOP_MAGICS", &magics)?;
    write_attacks(&mut output, "BISHOP_ATTACKS", &attacks)?;

    Ok(output)
}

fn generate_slider_tables(slider: Slider) -> ([MagicEntry; 64], Vec<u64>) {
    let mut magic_entries = [MagicEntry::default(); 64];
    let mut attack_table = Vec::with_capacity(match slider {
        Slider::Rook => 102_400,
        Slider::Bishop => 5_248,
    });
    let mut occupancies = Vec::with_capacity(match slider {
        Slider::Rook => 1 << 12,
        Slider::Bishop => 1 << 9,
    });
    let mut attacks = Vec::with_capacity(occupancies.capacity());

    let mut rng = SplitMix64::new(MAGIC_ROOT_SEED);

    for (square, entry) in magic_entries.iter_mut().enumerate() {
        let mask = relevant_mask(slider, square);
        let bits = mask.count_ones();
        let shift = 64 - bits;
        let occupancy_count = 1 << bits;

        occupancies.clear();
        let mut occupied: u64 = 0;
        for _ in 0..occupancy_count {
            occupancies.push(occupied);
            occupied = occupied.wrapping_sub(mask) & mask;
        }

        attacks.clear();
        attacks.extend(
            occupancies
                .iter()
                .map(|&occupied| sliding_attacks(slider, square, occupied)),
        );

        let (magic, _) = find_magic(&mut rng, mask, shift, &occupancies, &attacks);
        let offset = attack_table.len();
        attack_table.resize(offset + occupancy_count, 0);

        for (&occupied, &attack) in occupancies.iter().zip(&attacks) {
            attack_table[offset + magic_index(occupied, mask, magic, shift)] = attack;
        }

        *entry = MagicEntry {
            mask,
            magic,
            shift,
            offset,
        };
    }

    (magic_entries, attack_table)
}

fn find_magic(
    rng: &mut SplitMix64,
    mask: u64,
    shift: u32,
    occupancies: &[u64],
    attacks: &[u64],
) -> (u64, u64) {
    let table_size = 1 << mask.count_ones();
    let mut used = vec![0; table_size];
    let mut epoch = vec![0; table_size];

    let mut attempt = 0;

    loop {
        attempt += 1;
        let magic = rng.sparse_u64();
        if ((mask.wrapping_mul(magic)) & 0xff00_0000_0000_0000).count_ones() < 6 {
            continue;
        }

        let mut valid = true;

        for (&occupied, &attack) in occupancies.iter().zip(attacks) {
            let index = magic_index(occupied, mask, magic, shift);
            if epoch[index] != attempt {
                epoch[index] = attempt;
                used[index] = attack;
            } else if used[index] != attack {
                valid = false;
                break;
            }
        }

        if valid {
            return (magic, attempt);
        }
    }
}

fn relevant_mask(slider: Slider, square: usize) -> u64 {
    let rank = square as i32 / 8;
    let file = square as i32 % 8;
    let mut mask = 0;

    for (df, dr) in slider.directions() {
        let mut file = file + df;
        let mut rank = rank + dr;
        while (0..8).contains(&file) && (0..8).contains(&rank) {
            let next_file = file + df;
            let next_rank = rank + dr;
            if !(0..8).contains(&next_file) || !(0..8).contains(&next_rank) {
                break;
            }
            mask |= 1 << (rank * 8 + file);
            file += df;
            rank += dr;
        }
    }

    mask
}

fn sliding_attacks(slider: Slider, square: usize, occupied: u64) -> u64 {
    let rank = square as i32 / 8;
    let file = square as i32 % 8;
    let mut attacks = 0;

    for (df, dr) in slider.directions() {
        let mut file = file + df;
        let mut rank = rank + dr;
        while (0..8).contains(&file) && (0..8).contains(&rank) {
            let target = 1 << (rank * 8 + file);
            attacks |= target;
            if occupied & target != 0 {
                break;
            }
            file += df;
            rank += dr;
        }
    }

    attacks
}

fn magic_index(occupied: u64, mask: u64, magic: u64, shift: u32) -> usize {
    ((occupied & mask).wrapping_mul(magic) >> shift) as usize
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);

        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn sparse_u64(&mut self) -> u64 {
        self.next_u64() & self.next_u64() & self.next_u64()
    }
}

fn write_magics(output: &mut String, name: &str, entries: &[MagicEntry; 64]) -> fmt::Result {
    writeln!(output, "const {name}: [MagicEntry; 64] = [")?;
    for entry in entries {
        writeln!(output, "    {entry},")?;
    }
    output.push_str("];\n\n");
    Ok(())
}

fn write_attacks(output: &mut String, name: &str, attacks: &[u64]) -> fmt::Result {
    writeln!(output, "static {name}: [Bitboard; {}] = [", attacks.len())?;
    for attack in attacks {
        writeln!(output, "    Bitboard::new({attack:#018x}),")?;
    }
    output.push_str("];\n\n");
    Ok(())
}
