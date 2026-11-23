package com.example;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import java.io.Serializable;
import java.util.LinkedList;
import java.util.List;

public class LinkedListDemo implements Serializable {

  private static final long serialVersionUID = -2897624825673425478L;

  private int id;

  @JsonDeserialize(as = LinkedList.class)
  private List<String> names;
  @JsonDeserialize(as = LinkedList.class)
  private List<Float> scores;

  public int getId() {
	return id;
  }

  public void setId(int id) {
	this.id = id;
  }

  public List<String> getNames() {
	return names;
  }

  public void setNames(List<String> names) {
	this.names = names;
  }

  public List<Float> getScores() {
	return scores;
  }

  public void setScores(List<Float> scores) {
	this.scores = scores;
  }
}
