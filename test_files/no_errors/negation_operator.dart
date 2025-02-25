void main() {
  bool? boolean = maybeBool(true);
  boolean ??= true;
  if (!boolean) {
    print("false");
  }
  print("end");
}

bool? maybeBool(bool? boolean) {
  return boolean;
}
