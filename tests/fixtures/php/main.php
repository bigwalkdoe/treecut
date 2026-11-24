<?php

function greet($name) {
    return "hello $name";
}

class Animal {
    public function speak() {
        return "hi";
    }
}

interface Shape {
    public function area(): float;
}

trait Walkable {
    public function walk(): string {
        return "walking";
    }
}

enum Status: string {
    case Active = 'active';
    case Inactive = 'inactive';
}