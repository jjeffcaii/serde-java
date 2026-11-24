# https://just.systems

@test:
    cargo test --workspace

@ser-user:
    jbang Ser.java \
      -c examples/java/Address.java \
      -c examples/java/ExtInfo.java \
      -c examples/java/User.java \
      -n com.example.User \
      -j '{"id":123,"name":"Jack","age":18,"addresses":[{"country":"China","city":"Shanghai","street":"Dongfang Rd"},{"country":"China","city":"Beijing","street":"Changan Rd"}],"ext1":{"id":777,"key":"k777","value":"v777"},"ext2":{"id":888,"key":"k888","value":"v888"}}'


@ser-profile:
    jbang Ser.java \
      -c examples/java/Address.java \
      -c examples/java/Profile.java \
      -n com.example.Profile \
      -j '{"displayName":"Jack","nickname":"JJ","address":{"country":"China","city":"Shanghai","street":"Dongfang Rd"},"scores":[90,85,100]}'
