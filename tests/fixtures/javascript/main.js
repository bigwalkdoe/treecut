function greet(name) {
  return `hello ${name}`;
}

class Animal {
  speak() { return "hi"; }
}

const arrowFn = (x, y) => x + y;

export { greet, Animal };