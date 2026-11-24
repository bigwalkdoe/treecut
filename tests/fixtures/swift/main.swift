func greet(_ name: String) -> String {
    return "hello \(name)"
}

class Animal {
    func speak() -> String { "hi" }
}

struct Point {
    var x: Double
    var y: Double
}

protocol Shape {
    func area() -> Double
}

enum Color {
    case red, green, blue
}

init() {
    print("initializing")
}