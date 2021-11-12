public class Object {

    /*
    private static native void registerNatives();

    static {

        registerNatives();

    }
    */

    //public final native Class<?> getClass();
    public final Class<?> getClass() {return null; }

    //public native int hashCode();
    public int hashCode() { return 0; }

    public boolean equals(Object obj) {
        return (this == obj);
    }

    //protected native Object clone() throws CloneNotSupportedException;
    protected Object clone() throws CloneNotSupportedException {return null; }

    public String toString() {

        return getClass().getName() + "@" + Integer.toHexString(hashCode());

    }

    //public final native void notify();
    public final void notify() {}

    //public final native void notifyAll();
    public final void notifyAll() {};

    //public final native void wait(long timeout) throws InterruptedException;
    public final void wait(long timeout) throws InterruptedException {};


    public final void wait(long timeout, int nanos) throws InterruptedException {
        if (timeout < 0) {

            throw new IllegalArgumentException("timeout value is negative");


        }
        if (nanos < 0 || nanos > 999999) {

            throw new IllegalArgumentException(

                                "nanosecond timeout value out of range");

        }
        
        if (nanos >= 500000 || (nanos != 0 && timeout == 0)) {

            timeout++;

        }

        wait(timeout);
    }

    public final void wait() throws InterruptedException {
        wait(0);
    }

    protected void finalize() throws Throwable { }

}