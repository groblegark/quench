import { describe, it, expect } from 'vitest';

function multiply(a: number, b: number): number {
  return a * b;
}

describe('Math', () => {
  it('multiplies numbers', () => {
    expect(multiply(2, 3)).toBe(6);
  });
});
