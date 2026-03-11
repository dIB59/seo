export const normalizeUrl = (input: string) => {
  const trimmed = input.trim();
  try {
    const hasProtocol = /^https?:\/\//i.test(trimmed);
    const toTest = hasProtocol ? trimmed : `https://${trimmed}`;
    const parsed = new URL(toTest);
    return parsed.toString();
  } catch {
    return trimmed;
  }
};
