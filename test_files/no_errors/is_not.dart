void main() {
  var boolean = maybeBool(true);
  boolean ??= true;
  if (boolean is! bool) {
    print("false");
  }
  print("end");
}

bool? maybeBool(bool? boolean) {
  return boolean;
}
