export function transform(input: number, enabled: boolean): number {
  let value = input + 1;
  if (enabled) {
    value = value * 2;
  }
  return value;
}
