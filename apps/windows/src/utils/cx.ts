/** Join conditional class names; the only helper the UI layer needs. */
export function cx(
  ...values: (string | false | null | undefined)[]
): string {
  return values.filter(Boolean).join(" ");
}
