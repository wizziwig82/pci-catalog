# Testing Guide for Music Management App

This document provides guidance on how to run, manage, and extend the test suite for the Music Management App.

## Test Structure

The tests are organized into three main categories:

```
tests/
├── unit/                   # Unit tests for isolated components
│   ├── frontend/           # Frontend component tests
│   │   ├── components/     # Svelte component tests
│   │   └── utils/          # Utility function tests
│   └── backend/            # Backend functionality tests
│       ├── storage/        # R2 and MongoDB tests
│       ├── metadata/       # Metadata extraction tests
│       └── transcoding/    # Audio transcoding tests
├── integration/            # Integration tests for connected components
│   ├── credential-management/  # Credential storage tests
│   ├── upload-workflow/    # File upload workflow tests
│   ├── editing-workflow/   # Metadata editing tests
│   ├── search-functionality/ # Search feature tests
│   └── album-art-management/ # Album art workflow tests
└── e2e/                    # End-to-end tests
    ├── complete-workflows/ # Complete workflow tests
    └── website-integration/ # Next.js website integration tests
```

## Setting Up the Test Environment

1. Install the test dependencies:

```bash
npm install -D vitest @testing-library/svelte @testing-library/jest-dom jsdom c8
```

2. For frontend component testing:

```bash
npm install -D @sveltejs/vite-plugin-svelte
```

3. For Node.js functionality (used in some tests):

```bash
npm install -D @types/node
```

## Running Tests

### Running All Tests

```bash
npm test
```

This command runs all tests and displays a summary of the results.

### Running Specific Test Categories

To run just the unit tests:

```bash
npm run test:unit
```

To run just the integration tests:

```bash
npm run test:integration
```

To run just the end-to-end tests:

```bash
npm run test:e2e
```

### Running Tests in Watch Mode

During development, you may want to run tests in watch mode:

```bash
npm run test:watch
```

## Writing New Tests

### Unit Test Guidelines

- Each unit test should focus on testing a single function or component
- Mock external dependencies
- Keep tests fast and isolated

Example unit test for a Svelte component:

```typescript
import { render, fireEvent } from '@testing-library/svelte';
import { expect, test } from 'vitest';
import MyComponent from '../src/components/MyComponent.svelte';

test('component renders correctly', () => {
  const { getByText } = render(MyComponent, { props: { name: 'Test' } });
  expect(getByText('Hello, Test!')).toBeInTheDocument();
});
```

### Integration Test Guidelines

- Focus on how components work together
- Test workflows from end to end
- Mock external services but test real component interactions

### End-to-End Test Guidelines

- Test complete workflows from a user perspective
- Minimize mocking where possible
- Focus on critical user paths

## Test Fixtures

Test fixtures (like sample audio files) are stored in the `tests/fixtures` directory.

## Mocking

The app uses Vitest's mocking functionality to simulate external dependencies:

- Tauri API calls are mocked in `tests/setup.ts`
- R2 and MongoDB connections are mocked for unit tests
- File system operations use in-memory implementations for testing

## Coverage Reports

To generate a coverage report:

```bash
npm run test:coverage
```

The coverage report will be available in the `coverage` directory.

## Continuous Integration

Tests are automatically run in the CI pipeline. The testing workflow is defined in `.github/workflows/test.yml`.

## Best Practices

1. Write tests alongside implementation code
2. Keep tests simple and focused
3. Test both happy paths and error cases
4. Use descriptive test names
5. Clean up resources after tests complete
6. Keep tests fast to encourage running them frequently 