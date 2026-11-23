package com.example;

import java.io.Serializable;

public class BooleanDemo implements Serializable {

  private static final long serialVersionUID = 3043629198327648182L;

  private Boolean enabled;

  public Boolean getEnabled() {
	return enabled;
  }

  public void setEnabled(Boolean enabled) {
	this.enabled = enabled;
  }
}
