// Dummy module for SSR-only mode
export default function () {
  console.log("SSR-only mode - no client-side hydration");
  return Promise.resolve({});
}
