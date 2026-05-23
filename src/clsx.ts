export function clsx(...classes: readonly (string | false | undefined)[]) {
	return classes.filter(Boolean).join(" ")
}
