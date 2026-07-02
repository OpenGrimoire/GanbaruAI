export type MessageShape<T> = T extends (...args: infer Args) => string
  ? (...args: Args) => string
  : T extends string
  ? string
  : { readonly [Key in keyof T]: MessageShape<T[Key]> };
