package com.example;

import java.io.Serializable;

public class User implements Serializable {

  private static final long serialVersionUID = 4956385333250593913L;

  private long id;
  private String name;
  private int age;
  private Address[] addresses;

  private ExtInfo ext1;
  private ExtInfo ext2;

  public long getId() {
	return id;
  }

  public void setId(long id) {
	this.id = id;
  }

  public String getName() {
	return name;
  }

  public void setName(String name) {
	this.name = name;
  }

  public int getAge() {
	return age;
  }

  public void setAge(int age) {
	this.age = age;
  }

  public Address[] getAddresses() {
	return addresses;
  }

  public void setAddresses(Address[] addresses) {
	this.addresses = addresses;
  }

  public ExtInfo getExt1() {
	return ext1;
  }

  public void setExt1(ExtInfo ext1) {
	this.ext1 = ext1;
  }

  public ExtInfo getExt2() {
	return ext2;
  }

  public void setExt2(ExtInfo ext2) {
	this.ext2 = ext2;
  }
}
