package com.example;

import java.io.Serializable;
import com.example.Address;

public class Order implements Serializable {

  private static final long serialVersionUID = 2772851369020234932L;

  private int id;
  private Address address;

  public int getId() {
	return id;
  }

  public void setId(int id) {
	this.id = id;
  }

  public Address getAddress() {
	return address;
  }

  public void setAddress(Address address) {
	this.address = address;
  }

}
