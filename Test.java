public class Test {
    static double a;
    static int q;
    
    void foo() {
        a = 2;
        bar();
    }
    
    private void bar() {
        q = 5;
    }
}