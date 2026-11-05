// @ts-check
/**
 * ESLint flat config for the site: JavaScript and TypeScript recommended rules.
 * .astro files are type-checked by `astro check` (the `typecheck` script) rather than linted here.
 */
import js from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config({ ignores: ['dist/', '.astro/'] }, js.configs.recommended, tseslint.configs.recommended);
