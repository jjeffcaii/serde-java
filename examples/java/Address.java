package com.example;

import java.io.Serializable;

public class Address implements Serializable {

  private static final long serialVersionUID = -4433675896693646393L;

  private String country;
  private String city;
  private String street;

  public String getCountry() {
	return country;
  }

  public void setCountry(String country) {
	this.country = country;
  }

  public String getCity() {
	return city;
  }

  public void setCity(String city) {
	this.city = city;
  }

  public String getStreet() {
	return street;
  }

  public void setStreet(String street) {
	this.street = street;
  }
}
