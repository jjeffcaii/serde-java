package com.example;

import java.io.IOException;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;
import java.io.Serializable;

public class CustomPojo1 implements Serializable {

  private static final long serialVersionUID = -3231298442776514728L;

  private String username;
  private transient String password;
  private transient boolean admin;
  private transient boolean enabled;

  private void readObject(ObjectInputStream in) throws IOException, ClassNotFoundException {
  	in.defaultReadObject();
  	this.password = ((String) in.readObject()).substring("ENCRYPT_".length());
  	this.admin = in.readBoolean();
  	this.enabled = in.readBoolean();
  }

  private void writeObject(ObjectOutputStream out) throws IOException {
	out.defaultWriteObject(); // write default fields

	out.writeBoolean(admin);
	out.writeBoolean(enabled);

	out.writeObject("ENCRYPT_" + this.password); // write encrypted password manually
  }

  public String getUsername() {
	return username;
  }

  public void setUsername(String username) {
	this.username = username;
  }

  public String getPassword() {
	return password;
  }

  public void setPassword(String password) {
	this.password = password;
  }

  public boolean isAdmin() {
	return admin;
  }

  public void setAdmin(boolean admin) {
	this.admin = admin;
  }

  public boolean isEnabled() {
	return enabled;
  }

  public void setEnabled(boolean enabled) {
	this.enabled = enabled;
  }
}
