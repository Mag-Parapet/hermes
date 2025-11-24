import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FileStorages } from './file-storages';

describe('FileStorages', () => {
  let component: FileStorages;
  let fixture: ComponentFixture<FileStorages>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FileStorages]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FileStorages);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
