package com.example;

import java.io.Serializable;

public class PojoB implements Serializable {

  private static final long serialVersionUID = 311662489269302022L;

  private String name;

  public String getName() {
	return name;
  }

  public void setName(String name) {
	this.name = name;
  }

}
