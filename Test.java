public class Test {
    public final int x = 1;
    public static double y = 1.0;
    
    public static void main() {
        y = 5;
        var test = new Test();
		
        var test2 = new Test2();
        test2.y = 13;
        test.b(42, new Object(), test2);
        y = test2.foo(10);
    }
    
    public static void a(int q, Test2 r) {
        double x = 0;
		y = q;
		for (int i = 0; i < 10000; i++) {
			y--;
		}
        r.y += y;
        r.x += y;
    }
    
    public double b(int x, Object o, Test2 r) {
		y = x;
        this.a(x, r);
        
        return x + y;
    }

}