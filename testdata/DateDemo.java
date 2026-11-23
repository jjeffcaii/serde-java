package com.example;

import java.io.Serializable;
import java.util.Date;

public class DateDemo implements Serializable {

  private static final long serialVersionUID = 3088539499338478546L;
  private Date createdAt;

  public Date getCreatedAt() {
	return createdAt;
  }

  public void setCreatedAt(Date createdAt) {
	this.createdAt = createdAt;
  }

}
