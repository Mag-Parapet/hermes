import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
  name: 'unitConvert',
})
export class UnitConvertPipe implements PipeTransform {

  private units = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];

  transform(value: string | number): string {
    if (value == null) return '';

    let bytes: number;

    if (typeof value === 'number') {
      bytes = value;
    } else if (typeof value === 'string') {
      const match = value.trim().match(/^([\d,.]+)\s*([a-zA-Z]+)$/);
      if (!match) return value;

      const num = parseFloat(match[1].replace(/,/g, ''));
      const unit = match[2];

      const index = this.units.indexOf(unit);
      if (index === -1) return value;

      bytes = num * Math.pow(1024, index);
    } else {
      return '';
    }
    let converted = bytes;
    let index = 0;
    while (converted >= 1024 && index < this.units.length - 1) {
      converted /= 1024;
      index++;
    }

    return `${converted.toFixed(2)} ${this.units[index]}`;
  }

}
