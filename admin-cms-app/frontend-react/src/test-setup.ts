// Suppress React act() warnings in browser mode tests
// These warnings are benign - vitest-browser-react handles async with retry-ability
const originalError = console.error;
console.error = (...args: unknown[]) => {
  const message = typeof args[0] === "string" ? args[0] : "";
  if (message.includes("not wrapped in act")) {
    return;
  }
  originalError.apply(console, args);
};
