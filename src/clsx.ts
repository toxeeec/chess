export function clsx(...classes: readonly (string | false)[]) {
	return classes.filter(Boolean).join(" ")
}
