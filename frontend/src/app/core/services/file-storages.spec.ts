import { TestBed } from '@angular/core/testing';

import { FileStorages } from './file-storages';

describe('FileStorages', () => {
  let service: FileStorages;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(FileStorages);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
