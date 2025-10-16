# Frontend Testing Setup

This project uses Vitest for frontend unit testing with React Testing Library.

## Running Tests

```bash
# Run tests in watch mode
npm test

# Run tests once
npm run test:run

# Run tests with UI
npm run test:ui
```

## Test Structure

- Test files should be placed next to the component they test with `.test.tsx` extension
- Example: `Component.tsx` → `Component.test.tsx`

## Writing Tests

### Basic Component Test

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

### Mocking API Calls

```typescript
import { vi } from 'vitest';
import * as api from '../lib/api';

vi.mock('../lib/api', () => ({
  someApiCall: vi.fn(),
}));

// In your test
vi.mocked(api.someApiCall).mockResolvedValue({ data: 'test' });
```

### Testing User Interactions

```typescript
import { userEvent } from '@testing-library/user-event';

it('should handle click', async () => {
  const user = userEvent.setup();
  render(<Button />);
  
  await user.click(screen.getByRole('button'));
  expect(screen.getByText('Clicked')).toBeInTheDocument();
});
```

## Available Matchers

The setup includes `@testing-library/jest-dom` matchers:

- `toBeInTheDocument()`
- `toHaveTextContent()`
- `toBeVisible()`
- `toBeDisabled()`
- `toHaveClass()`
- And many more...

## Best Practices

1. **Test user behavior, not implementation details**
   - Use `screen.getByRole()` and `screen.getByLabelText()` over `getByTestId()`
   - Test what users see and do, not internal state

2. **Keep tests focused**
   - One assertion per test when possible
   - Test one behavior at a time

3. **Use async utilities properly**
   - Use `waitFor()` for async state changes
   - Use `findBy*` queries for elements that appear asynchronously

4. **Mock external dependencies**
   - Mock API calls
   - Mock Tauri commands
   - Keep tests isolated and fast

## Coverage

To generate coverage reports:

```bash
npm run test:run -- --coverage
```

Coverage reports will be generated in the `coverage/` directory.
