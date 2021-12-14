public class Test2 extends Test {
    static double a;
	public static void main() {
        new Test2().foo();
    }
    
    void foo() {
        a = 1;
        super.foo();

    }
}