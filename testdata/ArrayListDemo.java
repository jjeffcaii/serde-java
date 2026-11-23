package com.example;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import java.io.Serializable;
import java.util.ArrayList;
import java.util.List;

public class ArrayListDemo implements Serializable {

  private static final long serialVersionUID = 3153513349080412905L;

  private int id;
  @JsonDeserialize(as = ArrayList.class)
  private List<String> names;
  @JsonDeserialize(as = ArrayList.class)
  private List<Integer> scores;

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

  public List<Integer> getScores() {
	return scores;
  }

  public void setScores(List<Integer> scores) {
	this.scores = scores;
  }

}
