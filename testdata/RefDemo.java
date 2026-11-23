package com.example;

public class RefDemo implements java.io.Serializable {

  private static final long serialVersionUID = 8952467648642045509L;

  private long id;
  private String data;
  private RefDemo link;

  public long getId() {
	return id;
  }

  public void setId(long id) {
	this.id = id;
  }

  public RefDemo getLink() {
	return link;
  }

  public void setLink(RefDemo link) {
	this.link = link;
  }

  public String getData() {
	return data;
  }

  public void setData(String data) {
	this.data = data;
  }


}
