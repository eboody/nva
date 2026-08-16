import nextConfig from "eslint-config-next";

// Production verification leaves generated Next.js output in place. Keep repeated lint runs scoped
// to authored source instead of linting minified standalone bundles from a preceding build.
const config = [
  {
    ignores: [".next/**", "playwright-report/**", "test-results/**", "coverage/**"],
  },
  ...nextConfig,
];

export default config;
