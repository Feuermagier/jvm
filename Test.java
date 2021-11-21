public class Test {
    public final int x = 1;
    public static double y = 1.0;
    
    public static void main() {
        y = 5;
        var test = new Test();
		test.b(42, new Object());
    }
    
    public static void a(int q) {
        double x = 0;
		y = q;
		for (int i = 0; i < 10000; i++) {
			y--;
		}
    }
    
    public double b(int x, Object o) {
		y = x;
        this.a(x);
        
        return x + y;
    }

}