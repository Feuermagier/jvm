public class Test2 extends Test {
	public double y;
    static double a;
    static double b;
	public static void main() {
        new Test2().foo();
    }
    
    void foo() {
        
        super.y = 2;
        this.y = 1;
        a = this.y;
        b = super.y;
    }
}