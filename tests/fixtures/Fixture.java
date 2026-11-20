import java.io.*;
public class Fixture implements Serializable {
    private static final long serialVersionUID = 1L;
    String name = "fixture";
    short count = -123;
    int[] values = {1, -2, 3};
    String optional = null;
    public static void main(String[] args) throws Exception {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        new ObjectOutputStream(bytes).writeObject(new Fixture());
        for (byte b : bytes.toByteArray()) System.out.printf("%02x", b & 255);
    }
}
