package com.example;

import java.io.Serializable;

public class CharacterDemo implements Serializable {

  private static final long serialVersionUID = -1413488344517812718L;

  private Character ch;

  public Character getCh() {
	return ch;
  }

  public void setCh(Character ch) {
	this.ch = ch;
  }

}
