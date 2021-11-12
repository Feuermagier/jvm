public class Test {
    public final int x = 1;
    public static double y = 1.0;
    
    public static void main() {
        y = 10;
        var test = new Test();
    }
    
    public static void a() {
        double x = 0;
    }
    
    public double b() {
        this.a();
        
        return x + y;
    }

}