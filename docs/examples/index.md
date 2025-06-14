# Examples

This section contains practical examples of using Linen's reactivity system.

## Basic Counter

Let's start with a simple counter example that demonstrates the basics of signals, computed values, and effects:

```typescript
import { signal, computed, effect } from 'linen';

// Create a signal for the count
const count = signal(0);

// Create a computed value that doubles the count
const doubled = computed(() => count.value * 2);

// Create an effect that logs the count and doubled value
effect(() => {
  console.log(`Count: ${count.value}, Doubled: ${doubled.value}`);
});
// Logs: "Count: 0, Doubled: 0"

// Update the count
count.value = 5;
// Logs: "Count: 5, Doubled: 10"

// Update the count again
count.value = 10;
// Logs: "Count: 10, Doubled: 20"
```

## Form Validation

Here's a more complex example that shows how to use signals and computed values for form validation:

```typescript
import { signal, computed, effect } from 'linen';

// Create signals for form fields
const username = signal('');
const email = signal('');
const password = signal('');

// Create computed values for validation
const usernameError = computed(() => {
  if (!username.value) return 'Username is required';
  if (username.value.length < 3) return 'Username must be at least 3 characters';
  return '';
});

const emailError = computed(() => {
  if (!email.value) return 'Email is required';
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.value)) return 'Email is invalid';
  return '';
});

const passwordError = computed(() => {
  if (!password.value) return 'Password is required';
  if (password.value.length < 8) return 'Password must be at least 8 characters';
  return '';
});

// Create a computed value for form validity
const isFormValid = computed(() => 
  !usernameError.value && !emailError.value && !passwordError.value
);

// Log form state when it changes
effect(() => {
  console.log('Form state:');
  console.log(`Username: ${username.value} (${usernameError.value || 'valid'})`);
  console.log(`Email: ${email.value} (${emailError.value || 'valid'})`);
  console.log(`Password: ${password.value} (${passwordError.value || 'valid'})`);
  console.log(`Form is ${isFormValid.value ? 'valid' : 'invalid'}`);
});

// Simulate user input
username.value = 'jo';
// Form is invalid because username is too short

username.value = 'john';
// Username is now valid

email.value = 'invalid-email';
// Form is invalid because email is invalid

email.value = 'john@example.com';
// Email is now valid

password.value = 'short';
// Form is invalid because password is too short

password.value = 'password123';
// Password is now valid, form is valid
```

## Todo List State

This example shows how to manage a todo list using signals:

```typescript
import { signal, computed, effect } from 'linen';

// Define the Todo type
interface Todo {
  id: number;
  text: string;
  completed: boolean;
}

// Create a signal for the todo list
const todos = signal<Todo[]>([]);
const filter = signal<'all' | 'active' | 'completed'>('all');

// Create a computed value for filtered todos
const filteredTodos = computed(() => {
  const currentFilter = filter.value;
  
  if (currentFilter === 'all') {
    return todos.value;
  } else if (currentFilter === 'active') {
    return todos.value.filter(todo => !todo.completed);
  } else {
    return todos.value.filter(todo => todo.completed);
  }
});

// Create computed values for counts
const totalCount = computed(() => todos.value.length);
const activeCount = computed(() => todos.value.filter(todo => !todo.completed).length);
const completedCount = computed(() => todos.value.filter(todo => todo.completed).length);

// Log state when it changes
effect(() => {
  console.log(`Todos (${filteredTodos.value.length}):`);
  filteredTodos.value.forEach(todo => {
    console.log(`- [${todo.completed ? 'x' : ' '}] ${todo.text}`);
  });
  console.log(`Total: ${totalCount.value}, Active: ${activeCount.value}, Completed: ${completedCount.value}`);
});

// Add a todo
todos.value = [
  ...todos.value,
  { id: 1, text: 'Learn Linen', completed: false }
];

// Add another todo
todos.value = [
  ...todos.value,
  { id: 2, text: 'Build an app', completed: false }
];

// Toggle a todo
todos.value = todos.value.map(todo => 
  todo.id === 1 ? { ...todo, completed: true } : todo
);

// Filter todos
filter.value = 'active';
// Only shows uncompleted todos

filter.value = 'completed';
// Only shows completed todos

filter.value = 'all';
// Shows all todos
```