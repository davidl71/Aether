import js from '@eslint/js';
import globals from 'globals';
import parser from '@typescript-eslint/parser';
import tseslint from '@typescript-eslint/eslint-plugin';
import react from 'eslint-plugin-react';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';

const baseTsLanguageOptions = {
  parser,
  parserOptions: {
    project: './tsconfig.json',
    tsconfigRootDir: import.meta.dirname
  },
  globals: {
    ...globals.browser,
    ...globals.es2021
  }
};

const baseTsPlugins = {
  '@typescript-eslint': tseslint,
  react,
  'react-hooks': reactHooks,
  'react-refresh': reactRefresh
};

const baseTsRules = {
  ...js.configs.recommended.rules,
  ...tseslint.configs['recommended-type-checked'].rules,
  ...tseslint.configs['stylistic-type-checked'].rules,
  ...react.configs.recommended.rules,
  ...reactHooks.configs.recommended.rules,
  '@typescript-eslint/explicit-function-return-type': 'off',
  '@typescript-eslint/no-misused-promises': ['error', { checksVoidReturn: false }],
  'react-refresh/only-export-components': 'warn',
  'react/react-in-jsx-scope': 'off'
};

const reactSettings = {
  react: {
    version: 'detect'
  }
};

export default [
  {
    ignores: ['dist/**', 'node_modules/**']
  },
  {
    files: ['src/**/*.{ts,tsx}'],
    languageOptions: baseTsLanguageOptions,
    plugins: baseTsPlugins,
    rules: baseTsRules,
    settings: reactSettings
  },
  {
    files: ['src/**/*.{test,spec}.{ts,tsx}', 'src/**/__tests__/**/*.{ts,tsx}'],
    languageOptions: {
      ...baseTsLanguageOptions,
      globals: {
        ...baseTsLanguageOptions.globals,
        ...globals.jsdom,
        vi: 'readonly',
        describe: 'readonly',
        it: 'readonly',
        expect: 'readonly'
      }
    },
    plugins: baseTsPlugins,
    rules: baseTsRules,
    settings: reactSettings
  },
  {
    files: ['vite.config.ts', 'vitest.config.ts'],
    languageOptions: {
      parser,
      parserOptions: {
        project: './tsconfig.node.json',
        tsconfigRootDir: import.meta.dirname
      },
      globals: {
        ...globals.node,
        ...globals.es2021
      }
    },
    plugins: {
      '@typescript-eslint': tseslint
    },
    rules: {
      ...js.configs.recommended.rules,
      ...tseslint.configs['recommended-type-checked'].rules,
      ...tseslint.configs['stylistic-type-checked'].rules,
      '@typescript-eslint/explicit-function-return-type': 'off'
    }
  },
  {
    files: ['vitest.setup.ts'],
    languageOptions: {
      ...baseTsLanguageOptions,
      globals: {
        ...baseTsLanguageOptions.globals,
        ...globals.jsdom
      }
    },
    plugins: baseTsPlugins,
    rules: baseTsRules,
    settings: reactSettings
  }
];
