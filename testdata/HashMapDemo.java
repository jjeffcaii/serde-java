package com.example;

import java.io.Serializable;
import java.util.Map;
import java.util.HashMap;

public class HashMapDemo implements Serializable {

  private static final long serialVersionUID = -5707132217429457092L;

  private Map<String, String> exts;
  private Map<Integer, Double> scores;

  public Map<String, String> getExts() {
	return exts;
  }

  public void setExts(HashMap<String, String> exts) {
	this.exts = exts;
  }

  public Map<Integer, Double> getScores() {
	return scores;
  }

  public void setScores(HashMap<Integer, Double> scores) {
	this.scores = scores;
  }
}
