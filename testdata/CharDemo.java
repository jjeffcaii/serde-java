package com.example;

import java.io.Serializable;

public class CharDemo implements Serializable {

  private static final long serialVersionUID = -7957532738628518212L;

  private char ch;

  public char getCh() {
	return ch;
  }

  public void setCh(char ch) {
	this.ch = ch;
  }
}
