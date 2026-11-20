package com.example;

import java.io.Serializable;

public class Demo implements Serializable {

  private static final long serialVersionUID = 5151422842377556126L;

  private int i;
  private String message;

  public int getI() {
	return i;
  }

  public void setI(int i) {
	this.i = i;
  }

  public String getMessage() {
	return message;
  }

  public void setMessage(String message) {
	this.message = message;
  }
}
