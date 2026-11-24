export function greet(name: string): string {
  return `hello ${name}`;
}

export interface Foo { bar: number; }

export type Alias = string;

export class Animal {
  speak() { return "hi"; }
}
