public class Test {
    static double a;
	public static void main() {
		Test x = new Test();
        a = x.foo();
    }
    
    public int foo() {
        return 42;
    }
}