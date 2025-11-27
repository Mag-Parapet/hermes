import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FileDetails } from './file-details';

describe('FileDetails', () => {
  let component: FileDetails;
  let fixture: ComponentFixture<FileDetails>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [FileDetails]
    })
    .compileComponents();

    fixture = TestBed.createComponent(FileDetails);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
