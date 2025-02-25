void main() {
  bool? boolean = maybeBool(true);
  boolean ??= true;
  if (boolean != false) {
    print("false");
  }
  print("end");
}

bool? maybeBool(bool? boolean) {
  return boolean;
}
