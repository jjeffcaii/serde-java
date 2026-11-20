package com.example;

import java.io.Serializable;

public class Address implements Serializable {

  private static final long serialVersionUID = -4433675896693646393L;

  private String city;

  public String getCity() {
	return city;
  }

  public void setCity(String city) {
	this.city = city;
  }
}
