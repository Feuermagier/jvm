public class Test {
    public final int x = 1;
    public static double y = 1.0;
    
    public static void a() {
        double x = 0;
    }
    
    public double b() {
        a();
        return x + y;
    }

}