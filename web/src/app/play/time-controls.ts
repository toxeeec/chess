export const TIME_VALUES = [1, 2, 3, 5, 10, 15, 30] as const
export const INCREMENT_VALUES = [0, 1, 2, 3, 5, 10, 20] as const

export const DEFAULT_TIME = 10 satisfies (typeof TIME_VALUES)[number]
export const DEFAULT_INCREMENT = 0 satisfies (typeof INCREMENT_VALUES)[number]
