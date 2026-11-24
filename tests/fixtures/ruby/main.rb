def greet(name)
  "hello #{name}"
end

class Animal
  def speak
    "hi"
  end
end

module Walkable
  def walk
    "walking"
  end
end

class Person
  include Walkable
end