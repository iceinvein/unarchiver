# Testing Setup

This project now has Vitest configured for frontend unit testing.

## Quick Start

```bash
# Run tests in watch mode (interactive)
npm test

# Run tests once (CI mode)
npx vitest run

# Run tests with UI dashboard
npm run test:ui

# Run tests with coverage report
npm run test:coverage
```

## What's Included

### Testing Libraries
- **Vitest** - Fast unit test framework powered by Vite
- **@testing-library/react** - React component testing utilities
- **@testing-library/jest-dom** - Custom matchers for DOM assertions
- **@testing-library/user-event** - User interaction simulation
- **jsdom** - DOM implementation for Node.js

### Configuration Files
- `vitest.config.ts` - Vitest configuration with coverage settings
- `src/test/setup.ts` - Global test setup (matchers, cleanup)
- `src/test/utils.tsx` - Custom render utilities and re-exports
- `src/test/README.md` - Detailed testing guide

## Example Test

See `src/components/ArchivePreview.test.tsx` for a complete example that demonstrates:
- Component rendering
- Loading states
- Error handling
- Async operations with `waitFor`
- API mocking with `vi.mock`

## Writing Tests

Place test files next to the component they test:
```
src/components/
  ├── MyComponent.tsx
  └── MyComponent.test.tsx
```

Basic test structure:
```typescript
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import MyComponent from './MyComponent';

describe('MyComponent', () => {
  it('should render correctly', () => {
    render(<MyComponent />);
    expect(screen.getByText('Hello')).toBeInTheDocument();
  });
});
```

## Coverage

Coverage reports are generated in the `coverage/` directory and include:
- Text summary in terminal
- JSON report for CI integration
- HTML report for detailed browsing

The following are excluded from coverage:
- `node_modules/`
- `src/test/`
- Test files (`*.test.tsx`, `*.spec.tsx`)
- Generated bindings
- Rust code (`src-tauri/`)

## Best Practices

1. **Test user behavior** - Focus on what users see and do
2. **Use semantic queries** - Prefer `getByRole`, `getByLabelText` over `getByTestId`
3. **Mock external dependencies** - Keep tests fast and isolated
4. **One behavior per test** - Keep tests focused and maintainable
5. **Use async utilities** - `waitFor`, `findBy*` for async operations

## CI Integration

For CI/CD pipelines, use:
```bash
npm run test:run
```

This runs tests once and exits with appropriate status codes.

## Resources

- [Vitest Documentation](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/react)
- [Testing Library Best Practices](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library)
