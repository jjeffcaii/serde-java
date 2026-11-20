package com.example;

import java.io.Serializable;

public class Profile implements Serializable {

  private static final long serialVersionUID = 1;

  private String displayName;
  private String nickname;

  private Address address;
  private int[] scores;

  private transient String localCache;

  public String getDisplayName() {
	return displayName;
  }

  public void setDisplayName(String displayName) {
	this.displayName = displayName;
  }

  public String getNickname() {
	return nickname;
  }

  public void setNickname(String nickname) {
	this.nickname = nickname;
  }

  public Address getAddress() {
	return address;
  }

  public void setAddress(Address address) {
	this.address = address;
  }

  public int[] getScores() {
	return scores;
  }

  public void setScores(int[] scores) {
	this.scores = scores;
  }

  public String getLocalCache() {
	return localCache;
  }

  public void setLocalCache(String localCache) {
	this.localCache = localCache;
  }
}
