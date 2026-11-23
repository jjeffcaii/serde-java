package com.example;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import java.io.Serializable;
import java.util.Map;
import java.util.HashMap;

public class HashMapDemo implements Serializable {

  private static final long serialVersionUID = -5707132217429457092L;

  @JsonDeserialize(as = HashMap.class)
  private Map<String, String> exts;
  @JsonDeserialize(as = HashMap.class)
  private Map<Integer, Double> scores;

  public Map<String, String> getExts() {
	return exts;
  }

  public void setExts(Map<String, String> exts) {
	this.exts = exts;
  }

  public Map<Integer, Double> getScores() {
	return scores;
  }

  public void setScores(Map<Integer, Double> scores) {
	this.scores = scores;
  }
}
