public class Test2 extends Test {
	public int x = 2;
	public static void main() {
        var test = new Test2();
		test.x = 13;
		test.y = 42;
    }
	
	public void foo() {
		super.x = 3;
	}
}