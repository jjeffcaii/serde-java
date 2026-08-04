package com.example;

import java.io.Serializable;
import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@AllArgsConstructor
public class User implements Serializable {

  private static final long serialVersionUID = 4956385333250593913L;

  private long id;
  private String name;
  private int age;
  private Address[] addresses;

  private ExtInfo ext1;
  private ExtInfo ext2;
}

@Data
@AllArgsConstructor
@NoArgsConstructor
public class ExtInfo implements Serializable {

  private static final long serialVersionUID = 8520976260072537200L;

  private int id;
  private String key;
  private String value;

}

@Data
@AllArgsConstructor
@NoArgsConstructor
public class Address implements Serializable {

  private static final long serialVersionUID = -4433675896693646393L;

  private String country;
  private String city;
  private String street;

}
