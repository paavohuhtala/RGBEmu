
trait Mapper<T> {
  fn write_8(location: T, value: u8);
}

struct BaseMapper {

}

enum BaseMapperLocation {

}

enum MBC1Location {

}

impl Mapper<BaseMapperLocation> for BaseMapper {
  fn write_8(location: BaseMapperLocation, value: u8) {

  }
}

struct MBC1 {

}

impl Mapper<MBC1Location> for MBC1 {
  fn write_8(location: MBC1Location, value: u8) {

  }
}
