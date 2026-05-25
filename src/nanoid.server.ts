const SIZE = 21
const ALPHABET = "useandom-26T198340PX75pxJACKVERYMINDBUSHWOLF_GQZbfghjklqvwyzrict"

export function nanoid() {
	let id = ""
	const bytes = crypto.getRandomValues(new Uint8Array(SIZE))

	for (let i = 0; i < SIZE; ++i) {
		id += ALPHABET[bytes[i]! & 63]
	}

	return id
}
