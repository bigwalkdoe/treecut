using System;

namespace Demo
{
    public class Animal
    {
        public void Speak() { }
        public Animal() { }
    }

    public interface IShape { double Area(); }

    public struct Point { public int X; public int Y; }

    public enum Color { Red, Green, Blue }

    public record Person(string Name, int Age);

    public static class Utils
    {
        public static int Add(int a, int b) => a + b;
    }
}